//! The report tail carved from `runner.rs` (issue 098 T2): the table
//! formatters (`fmt4` / `fmt_opt` / `fmt_gate_fit`), the JDI crosswalk
//! predicate, and [`render_markdown`] - the run's Markdown report. Bodies
//! moved verbatim; `runner.rs` re-exports `render_markdown` so the
//! `harness` bin's path is unchanged.

#![allow(clippy::too_many_lines)]

use super::*;
// ── table rendering ─────────────────────────────────────────────────────

fn fmt4(x: f64) -> String {
    format!("{x:.4}")
}

fn fmt_opt(x: Option<f64>) -> String {
    x.map_or("—".to_string(), fmt4)
}

/// Render the run as markdown (`TABLES.md`).
#[must_use]
/// The gate-fit table cell (Issue 009 T2/T3): the fitted pair + the
/// support disclosure, `thin → defaults` when the cal slice was too small,
/// `—` on lanes that fit no gates.
fn fmt_gate_fit(r: &LaneResult) -> String {
    let Some(rec) = &r.threshold_recommendation else {
        return "—".to_string();
    };
    match (&rec.score, &rec.distance) {
        (Some(s), Some(d)) => format!(
            "s {} / d {} (n {})",
            fmt4(f64::from(s.threshold)),
            fmt4(f64::from(d.threshold)),
            s.support.n
        ),
        _ => "thin → defaults".to_string(),
    }
}

/// Plan 011 §B5: the JDI crosswalk renders for the DATASET population
/// only — the registry rows that read real dataset files at the protocol
/// caps (not the six in-process harness families — Issue 059's quarantine
/// — and not the s1mb named-only protocol suites, whose variable option
/// sets make an index-class chance a label-chance lie). Membership is read
/// off the SUITES registry (synthetic: None ∧ not named-only), never a
/// hand-typed name list — a dataset suite joining the registry joins the
/// crosswalk by existing.
fn jdi_crosswalk_applies(name: &str) -> bool {
    SUITES
        .iter()
        .any(|sp| sp.name == name && sp.synthetic.is_none() && !sp.named_only)
}

pub fn render_markdown(out: &RunOutput, errors: &[String]) -> String {
    let mut s = String::new();
    s.push_str("# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)\n\n");
    s.push_str(&format!(
        "- run: `{}` on `{}` ({}) · profile {} · laya feature {}\n",
        out.meta.git_sha, out.meta.host, out.meta.date_utc, out.meta.profile, out.meta.laya_feature
    ));
    if out.meta.laya_feature {
        // The 008 T4 move landed (2026-09-24, `src/laya` → the riir-infer
        // repo's `riir-infer-laya` crate): the PRE-MOVE-BASELINE caveat is
        // retired — the sha above now pins the post-move tree, whose laya
        // substrate is a path dep versioned in the substrate repo.
        if out.meta.laya_max_questions > 0 {
            s.push_str(&format!(
                "- laya cap: {} questions per checkpoint — PARTIAL run; the laya `n` columns \
                 reflect the cap and are NOT comparable row-wise against the modelless `n` columns\n",
                out.meta.laya_max_questions
            ));
        }
    }
    s.push_str(&crate::harness::box_state::render_line(&out.meta.box_state));
    s.push_str(&format!(
        "- laya device posture: {}\n",
        out.meta.laya_device
    ));
    s.push_str(&format!(
        "- laya-python lane: {}\n",
        out.meta.laya_python_lane
    ));
    s.push_str(&format!("- clm lane: {}\n", out.meta.clm_lane));
    s.push_str(&format!("- gliner lane: {}\n", out.meta.gliner_lane));
    s.push_str(&format!("- bekko lane: {}\n", out.meta.bekko_lane));
    s.push_str(&format!("- agentjev lane: {}\n", out.meta.agentjev_lane));
    s.push_str(&format!("- clef lane: {}\n", out.meta.clef_lane));
    s.push_str(&format!("- drex lane: {}\n", out.meta.drex_lane));
    s.push_str(&format!("- d1 lane: {}\n", out.meta.d1_lane));
    s.push_str(&format!("- pplx lane: {}\n", out.meta.pplx_lane));
    s.push_str(&format!("- d3 lane: {}\n", out.meta.d3_lane));
    s.push_str(&format!("- startlux lane: {}\n", out.meta.startlux_lane));
    s.push_str(&format!("- mammoth lane: {}\n", out.meta.mammoth_lane));
    s.push_str(&format!("- elephant lane: {}\n", out.meta.elephant_lane));
    s.push_str(&format!("- paw lane: {}\n", out.meta.paw_lane));
    s.push_str(&format!("- paw-local lane: {}\n", out.meta.paw_local_lane));
    s.push_str(&format!(
        "- corpus cap posture: {}\n",
        out.meta.corpus_cap_mode
    ));
    s.push_str(&format!("- label heads: {}\n", out.meta.head_posture));
    s.push_str(&format!(
        "- count tables (issue 038): {}\n",
        out.meta.nb_posture
    ));
    #[cfg(feature = "option_cond")]
    s.push_str(&format!(
        "- option-conditioned tables (issue 038 T7b): {}\n",
        out.meta.oc_posture
    ));
    #[cfg(feature = "nb_ridge")]
    s.push_str(&format!(
        "- NBSVM ridge readout (issue 038 T7a): {}\n",
        out.meta.ridge_posture
    ));
    s.push_str(&format!(
        "- corpus: {} \n- calibration: {}\n- sampling: {}\n- readout: {}\n- floor: {}\n- modal floor: {}\n- determinism: {}\n",
        out.meta.corpus_protocol,
        out.meta.calibration_protocol,
        out.meta.sampling_protocol,
        out.meta.readout_posture,
        out.meta.floor_definition,
        out.meta.modal_floor_definition,
        out.meta.determinism_scoping
    ));
    for d in &out.meta.divergences {
        s.push_str(&format!("- divergence: {d}\n"));
    }
    s.push('\n');

    if !errors.is_empty() {
        s.push_str("## Absences / errors (honest — never silently dropped)\n\n");
        for e in errors {
            s.push_str(&format!("- {e}\n"));
        }
        s.push('\n');
    }

    for suite in &out.suites {
        s.push_str(&format!(
            "## {} — {} cases / {} questions\n\n",
            suite.name, suite.n_cases, suite.n_questions
        ));
        // Issue 058: the slice-integrity record beside every suite's
        // numbers — counts, raw slice digests, and any disclosed label
        // starvation. A pool digest that moved between runs is the "the
        // corpus moved, not the engine" tell (Bench 076's class). Violations
        // cannot appear here (prepare refuses them before any lane runs);
        // the disclosed starvation fields are what this line can carry.
        if let Some(sl) = &suite.slices {
            s.push_str(&format!("**{}**\n\n", sl.summary_line()));
        }
        // The corpus-cap disclosure + the cal-slice selection table when
        // selection ran (Issue 013 lever-1 protocol): the selection reads
        // cal only; the suite's row below IS the single test read.
        if let Some(m) = &suite.modelless
            && let Some(cap) = &m.corpus_cap
        {
            s.push_str(&format!(
                "**corpus cap:** {} ({})\n\n",
                cap.effective, cap.source
            ));
            if let Some(rows) = &cap.selection {
                s.push_str("| cap | cal acc |\n|---|---|\n");
                for c in rows {
                    let mark = if c.cap == cap.effective {
                        " ← selected"
                    } else {
                        ""
                    };
                    s.push_str(&format!("| {}{} | {} |\n", c.cap, mark, fmt4(c.cal_acc)));
                }
                s.push('\n');
            }
        }
        // The fitted-head disclosure (issue 030 lever 4): the selected
        // scale + the per-candidate cal accuracies when selection ran.
        if let Some(m) = &suite.modelless
            && let Some(hs) = &m.head_selection
        {
            s.push_str(&format!(
                "**label heads:** cal-selected scale {} (ladder 0/0.25/0.5/1, forced cal \
                 accuracy, ties → 0)\n\n",
                hs.selected
            ));
            s.push_str("| head scale | cal acc |\n|---|---|\n");
            for c in &hs.candidates {
                let mark = if c.scale == hs.selected {
                    " ← selected"
                } else {
                    ""
                };
                s.push_str(&format!("| {}{} | {} |\n", c.scale, mark, fmt4(c.cal_acc)));
            }
            s.push('\n');
        }
        // The count-table disclosure (issue 038) + the transductive column,
        // printed apart from the headline row on purpose.
        if let Some(m) = &suite.modelless
            && let Some(ns) = &m.nb_selection
        {
            s.push_str(&format!(
                "**count tables:** cal-selected scale {} α {} view {} (promotion bar +5 pt over \
                 off)\n\n",
                ns.selected_scale, ns.selected_alpha, ns.selected_view
            ));
            s.push_str(
                "| nb scale | α | noul yes→domain | view | cal acc |\n|---|---|---|---|---|\n",
            );
            for c in &ns.candidates {
                let mark = if c.scale == ns.selected_scale
                    && c.alpha == ns.selected_alpha
                    && c.noul_domain == ns.selected_noul_domain
                    && c.view == ns.selected_view
                {
                    " ← selected"
                } else {
                    ""
                };
                s.push_str(&format!(
                    "| {}{} | {} | {} | {} | {} |\n",
                    c.scale,
                    mark,
                    c.alpha,
                    c.noul_domain.map_or("—".to_string(), |d| d.to_string()),
                    c.view,
                    fmt4(c.cal_acc)
                ));
            }
            s.push('\n');
        }
        // The blend-genome disclosure (issue 038 T5): the joint walk over
        // the composed posture's scale coordinates, printed apart from the
        // headline row like the selections above it.
        if let Some(m) = &suite.modelless
            && let Some(gs) = &m.genome_selection
        {
            let pt = |p: &genome_lane::GenomePoint| {
                #[cfg(feature = "nb_scope")]
                let nb = format!(" nb {} α {} view {}", p.nb_scale, p.nb_alpha, p.nb_view);
                #[cfg(not(feature = "nb_scope"))]
                let nb = String::new();
                #[cfg(feature = "option_cond")]
                let oc = format!(" oc {}", p.oc_scale);
                #[cfg(not(feature = "option_cond"))]
                let oc = String::new();
                #[cfg(feature = "nb_ridge")]
                let ridge = format!(" ridge {}", p.ridge_scale);
                #[cfg(not(feature = "nb_ridge"))]
                let ridge = String::new();
                format!(
                    "route {} head {}{}{}{} (cal {})",
                    p.route_scale,
                    p.head_scale,
                    nb,
                    oc,
                    ridge,
                    fmt4(p.cal_acc)
                )
            };
            s.push_str(&format!(
                "**blend genome:** {} — {} ({} pass(es), promotion bar +5 pt over the seed)

",
                pt(&gs.selected),
                if gs.held {
                    "HELD — the composed posture stands"
                } else {
                    "ACCEPTED"
                },
                gs.passes
            ));
            if !gs.moves.is_empty() {
                s.push_str("| pass | coordinate | from | to | cal acc |\n|---|---|---|---|---|\n");
                for mv in &gs.moves {
                    s.push_str(&format!(
                        "| {} | {} | {} | {} | {} |\n",
                        mv.pass,
                        mv.coord,
                        mv.from,
                        mv.to,
                        fmt4(mv.cal_acc)
                    ));
                }
                s.push('\n');
            }
        }
        if let Some(m) = &suite.modelless
            && let Some(t) = &m.transductive
        {
            s.push_str(&format!(
                "**transductive column (NOT the headline):** acc {} vs honest {} ({:+.1} pt; \
                 {} pseudo-labelled test docs) — {}\n\n",
                fmt4(t.accuracy),
                fmt4(t.honest_accuracy),
                (t.accuracy - t.honest_accuracy) * 100.0,
                t.n_pseudo,
                t.protocol
            ));
        }
        // Issue 039 T4/T3 disclosures: the readout candidate report (the
        // arming lever is demoted — recorded when a candidate would have
        // won on cal) and any label the corpus pool starved.
        if let Some(m) = &suite.modelless
            && let Some(rs) = &m.readout_report
            && rs.best_on_cal != "dispatch"
        {
            let cands = rs
                .candidates
                .iter()
                .map(|c| format!("{} {:.4}", c.mode, c.cal_ece))
                .collect::<Vec<_>>()
                .join(" · ");
            s.push_str(&format!(
                "**readout (report-only):** best-on-cal `{}` NOT armed — the margin pick overfits \
                 cal (Bench 052 demotion); cal in-sample calibrated ECE: {cands}\n\n",
                rs.best_on_cal
            ));
        }
        if let Some(m) = &suite.modelless
            && !m.corpus_fallbacks.is_empty()
        {
            s.push_str(&format!(
                "⛔ **corpus fallback (Issue 039 guard):** {} option label(s) with NO train docs \
                 in the corpus pool — self-doc fallback only: {}\n\n",
                m.corpus_fallbacks.len(),
                m.corpus_fallbacks.join(", ")
            ));
        }
        // Issue 094 T1 — the input-keyed modal-answer floor: the
        // anti-mimicry census every lane row below is quoted against
        // (report-only; a lane at/below the floor carries no
        // input-independent signal and the block names it).
        if let Some(f) = &suite.modal_floor {
            let mut entries: Vec<(&str, &str, f64)> = Vec::new();
            if let Some(m) = &suite.modelless {
                entries.push((m.lane, &m.model, m.hard.accuracy));
            }
            for r in suite
                .laya
                .values()
                .chain(suite.clm.iter())
                .chain(suite.gliner.iter())
                .chain(suite.bekko.iter())
            {
                entries.push((r.lane, &r.model, r.hard.accuracy));
            }
            s.push_str(&crate::harness::modal_floor::render_block(&entries, f));
        }
        let Some(m) = &suite.modelless else {
            if suite.laya.is_empty() {
                s.push_str(
                    "> modelless lane: SKIPPED — LLM-lane only (Issue 004 T3: the \
                     modelless lane has no KV cache; compile with `--features laya-riir`).\n\n",
                );
            } else {
                s.push_str(
                    "> modelless lane: SKIPPED — LLM-lane only (the modelless lane has no \
                     KV cache, so it has no honest answer for this family); the laya lane \
                     answered below.\n\n",
                );
            }
            if !suite.laya.is_empty()
                || suite.clm.is_some()
                || suite.gliner.is_some()
                || suite.bekko.is_some()
            {
                s.push_str("| lane · model | n | acc | ECE(maxp) | readout-ECE | p50 | p99 (support) | det |\n");
                s.push_str("|---|---|---|---|---|---|---|---|\n");
                for r in suite
                    .laya
                    .values()
                    .chain(suite.clm.iter())
                    .chain(suite.gliner.iter())
                    .chain(suite.bekko.iter())
                {
                    s.push_str(&format!(
                        "| {} · {} | {} | {} | {} | {} | {:.1} ms | {} | {} |\n",
                        r.lane,
                        r.model,
                        r.hard.n,
                        fmt4(r.hard.accuracy),
                        fmt4(r.hard.ece),
                        fmt_opt(r.readout_ece),
                        r.latency_p50_ms,
                        fmt_p99_cell(
                            r.latency_p99_ms,
                            r.latency_tail_support,
                            r.latency_extremes.as_ref(),
                            1
                        ),
                        r.determinism_ok
                            .map_or("—", |ok| if ok { "✓" } else { "✗" }),
                    ));
                }
            } else {
                s.push_str("(no lane produced a result — see the absences section)\n");
            }
            s.push('\n');
            continue;
        };
        s.push_str(
            "| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |\n",
        );
        s.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
        s.push_str(&format!(
            "| modelless | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2}/{:.2} | {} | {:.3} ms | {} | {} | {} |\n",
            m.hard.n,
            fmt4(m.hard.accuracy),
            fmt4(m.hard.macro_f1),
            fmt4(m.hard.ece),
            fmt4(m.hard.brier),
            fmt4(m.hard.nll),
            fmt4(m.hard.aurc),
            fmt4(m.hard.acc_at_50_coverage),
            fmt_opt(m.readout_ece),
            m.raw_abstain.as_ref().map_or(0.0, |a| a.abstain_rate),
            m.calibrated_abstain.as_ref().map_or(0.0, |a| a.abstain_rate),
            m.calibrated_abstain
                .as_ref()
                .map_or("—".to_string(), |a| fmt4(a.selective_accuracy)),
            m.latency_p50_ms,
            fmt_p99_cell(
                m.latency_p99_ms,
                m.latency_tail_support,
                m.latency_extremes.as_ref(),
                3
            ),
            m.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            fmt_gate_fit(m),
        ));
        for r in suite.laya.values() {
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if !suite.cascade.is_empty() {
            s.push_str("\n**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.\n\n");
            s.push_str("| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |\n");
            s.push_str("|---|---|---|---|---|---|---|---|---|\n");
            for (ck, rep) in &suite.cascade {
                let esc_den = rep.n_escalated;
                s.push_str(&format!(
                    "| cascade · {} | {} | {:.1}% | {} | {} | {:+} | {} | {} | {} |\n",
                    ck,
                    rep.n_questions,
                    rep.escalation_rate * 100.0,
                    fmt4(rep.accuracy),
                    fmt4(m.hard.accuracy),
                    format!("{:+.4}", rep.accuracy - m.hard.accuracy),
                    if esc_den > 0 {
                        format!(
                            "{} / {} ({})",
                            rep.escalated_laya_correct,
                            esc_den,
                            fmt4(rep.escalated_laya_correct as f64 / esc_den as f64)
                        )
                    } else {
                        "—".to_string()
                    },
                    if esc_den > 0 {
                        format!(
                            "{} / {} ({})",
                            rep.escalated_modelless_correct,
                            esc_den,
                            fmt4(rep.escalated_modelless_correct as f64 / esc_den as f64)
                        )
                    } else {
                        "—".to_string()
                    },
                    if rep.n_un_escalated > 0 {
                        let why = if rep.worthiness.as_ref().is_some_and(|w| w.disarmed) {
                            "disarmed by worthiness"
                        } else {
                            "coverage limit"
                        };
                        format!("{} ({})", rep.n_un_escalated, why)
                    } else {
                        "0".to_string()
                    },
                ));
            }
            s.push('\n');
            // Issue 042 lever 3: the worthiness verdicts, when the lever
            // ran — one row per checkpoint, the cal-slice probe read that
            // armed or disarmed the escalation above.
            if suite.cascade.values().any(|r| r.worthiness.is_some()) {
                s.push_str("\n**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.\n\n");
                s.push_str("| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |\n");
                s.push_str("|---|---|---|---|---|---|---|---|---|\n");
                for (ck, rep) in &suite.cascade {
                    let Some(w) = &rep.worthiness else {
                        continue;
                    };
                    let lcb_cell = w
                        .probe_lcb
                        .map(|v| format!("{v:+.4}"))
                        .unwrap_or_else(|| "—".to_string());
                    s.push_str(&format!(
                        "| cascade · {} | {} | {} | {} | {} | {:+.4} | {} | {:+.4} | {} |\n",
                        ck,
                        if w.disarmed { "DISARMED" } else { "armed" },
                        w.probe_n,
                        fmt4(w.probe_laya_acc),
                        fmt4(w.probe_modelless_acc),
                        w.delta,
                        lcb_cell,
                        w.min_delta,
                        w.unprobed_reason.as_deref().unwrap_or("—"),
                    ));
                }
                s.push('\n');
            }
        }
        if let Some(r) = &suite.clm {
            // Same shape as a laya row — the CLM reference is a comparison
            // lane with the same metrics surface (no abstain, no gates).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.gliner {
            // Same shape as the clm row — the GLiNER reference is a
            // comparison lane with the same metrics surface (no abstain,
            // no gates; latency = subprocess round-trip, the laya-python
            // measurement law).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.bekko {
            // Same shape as the gliner row — the Bekko reference is a
            // comparison lane with the same metrics surface (no abstain,
            // no gates; latency = subprocess round-trip, the laya-python
            // measurement law).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.agentjev {
            // Same shape as the gliner row — the AgentJev reference is a
            // comparison lane with the same metrics surface (no abstain,
            // no gates; latency = client round-trip over HTTP).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.clef {
            // Same shape as the agentjev row — the Clef reference is a
            // comparison lane with the same metrics surface (latency =
            // client round-trip INCLUDING the disclosed forwarder hop,
            // plan 011 A0; the wire fixture A2.5 gates any published cell).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.openthai {
            // Same shape as the agentjev row — the OpenThai reference is a
            // comparison lane with the same metrics surface (their abstain
            // slots are provenance, not our selective metrics; latency =
            // client round-trip over HTTP).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.drex {
            // Same shape as the openthai row — the Drex reference is a
            // comparison lane with the same metrics surface (readout_ece
            // here is the comparable ece(maxp) column; THEIR confidence
            // fields get the dedicated detail line below — never folded
            // into this column).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.d1 {
            // Same shape as the drex row — the d1 reference is a
            // comparison lane with the same metrics surface (Issue 078).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.pplx {
            // Same shape as the d1 row — the pplx-decider reference is a
            // comparison lane with the same metrics surface (Issue 082;
            // latency = client round-trip INCLUDING the disclosed network
            // hop to the serve box; determinism = the verbatim no-clock
            // observed-repeat).
            s.push_str(&format!(
                "| {} · {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.hard.n,
                fmt4(r.hard.accuracy),
                fmt4(r.hard.macro_f1),
                fmt4(r.hard.ece),
                fmt4(r.hard.brier),
                fmt4(r.hard.nll),
                fmt4(r.hard.aurc),
                fmt4(r.hard.acc_at_50_coverage),
                fmt_opt(r.readout_ece),
                r.latency_p50_ms,
                fmt_p99_cell(
                    r.latency_p99_ms,
                    r.latency_tail_support,
                    r.latency_extremes.as_ref(),
                    1
                ),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.paw {
            // Free text, no probability surface: accuracy + latency only
            // (refusals scored wrong); the refusal line follows the table.
            s.push_str(&format!(
                "| {} · {} | {} | {} | — | — | — | — | — | — | — | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.n_questions,
                fmt4(r.accuracy),
                r.latency_p50_ms,
                fmt_p99_cell(r.latency_p99_ms, r.latency_tail_support, None, 1),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if let Some(r) = &suite.paw_local {
            // The runtime-posture twin of the paw row: same columns, local
            // latency (no network), determinism promised by construction
            // and re-verified per run.
            s.push_str(&format!(
                "| {} · {} | {} | {} | — | — | — | — | — | — | — | — / — | — | {:.1} ms | {} | {} | — |\n",
                r.lane,
                r.model,
                r.n_questions,
                fmt4(r.accuracy),
                r.latency_p50_ms,
                fmt_p99_cell(r.latency_p99_ms, r.latency_tail_support, None, 1),
                r.determinism_ok.map_or("—", |ok| if ok { "✓" } else { "✗" }),
            ));
        }
        if suite.laya.is_empty() {
            s.push_str("| laya · (absent) | — the laya lane did not run for this suite (feature off / weights missing) — see absences above |\n");
        }
        if let Some(r) = &suite.paw {
            s.push_str(&crate::lanes::paw::render_detail_line(r));
        }
        if let Some(r) = &suite.paw_local {
            s.push_str(&crate::lanes::paw::render_detail_line(r));
        }
        // Issue 076 T2: the comparison lanes' confidence-vs-correctness
        // cells — THEIR wire `confidence` fields audited (drex's card
        // disclaims calibratedness, d1's card claims it), printed apart
        // from the comparable ece(maxp) column on purpose. One render
        // path for every lane carrying the shared readout.
        let push_conf_cells =
            |s: &mut String, label: &str, cr: &crate::lanes::drex::DrexConfReadout| {
                let cell = |name: &str, c: &Option<crate::lanes::drex::DrexConfCell>| match c {
                    Some(c) => format!(
                        "{name} n={} ece={} brier={}",
                        c.n,
                        fmt4(c.ece),
                        fmt4(c.brier)
                    ),
                    None => format!("{name} absent (no wire confidence)"),
                };
                let floor = cr.floor_ece.map_or_else(
                    || "below the occupancy floor (n < 40)".to_string(),
                    |e| format!("ece={}", fmt4(e)),
                );
                s.push_str(&format!(
                "\n**{label} their-confidence cells:** {} · {} · floor {} (cal {} / test {}; {})\n\n",
                cell("choice", &cr.choice),
                cell("score", &cr.score),
                floor,
                cr.floor_n_cal,
                cr.floor_n_test,
                cr.floor_posture,
            ));
            };
        if let Some(cr) = suite
            .drex
            .as_ref()
            .and_then(|r| r.drex_conf_readout.as_ref())
        {
            push_conf_cells(&mut s, "drex", cr);
        }
        if let Some(cr) = suite.d1.as_ref().and_then(|r| r.drex_conf_readout.as_ref()) {
            // For d1 the wire confidence IS the max probability — the
            // cells restate the ece(maxp) axis per kind + add the floor
            // companion; the label names the lane so the shared field's
            // historical spelling never misleads a reader.
            push_conf_cells(&mut s, "d1", cr);
        }

        // Plan 011 §B3–B5: the JDI-protocol crosswalk — chance + skill
        // columns for every lane with forced picks on this suite, rendered
        // ONLY for the dataset population (the harness families stay
        // quarantined from any JDI column — Issue 059 law; the s1mb
        // named-only protocol suites and synthetic builds carry variable
        // option sets where an index-class chance is not a label chance).
        // B4's coverage law is structural here: a lane row exists only if
        // EVERY case was answered (the harness refuses-and-fails-loud),
        // so the coverage columns are constants, disclosed in the header
        // rather than fabricated per row.
        if jdi_crosswalk_applies(&suite.name) {
            let mut rows: Vec<&LaneResult> = Vec::new();
            if let Some(m) = &suite.modelless {
                rows.push(m);
            }
            rows.extend(suite.laya.values());
            for r in [
                suite.clm.as_ref(),
                suite.gliner.as_ref(),
                suite.bekko.as_ref(),
                suite.agentjev.as_ref(),
                suite.clef.as_ref(),
                suite.openthai.as_ref(),
                suite.drex.as_ref(),
                suite.d1.as_ref(),
                suite.pplx.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                rows.push(r);
            }
            if !rows.is_empty() {
                s.push_str(
                    "\n**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — \
                     different corpus, protocol caps, hardware; board membership requires their full frozen \
                     suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — \
                     the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) \
                     clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness \
                     law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families \
                     carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, \
                     not lanes).\n\n",
                );
                s.push_str(
                    "| lane · model | n | acc | macro F1 | chance | skill | det |\n|---|---|---|---|---|---|---|\n",
                );
                for r in &rows {
                    s.push_str(&format!(
                        "| {} · {} | {} | {} | {} | {} | {} | {} |\n",
                        r.lane,
                        r.model,
                        r.hard.n,
                        fmt4(r.hard.accuracy),
                        fmt4(r.hard.macro_f1),
                        fmt_opt(r.jdi_chance),
                        fmt_opt(r.jdi_skill),
                        r.determinism_ok
                            .map_or("—", |ok| if ok { "✓" } else { "✗" }),
                    ));
                }
                s.push('\n');
            }
        }

        // G1 table (modelless only).
        if let (Some(raw), Some(cal), Some(floor), Some(verdict)) = (
            m.readout_ece_raw,
            m.readout_ece_calibrated,
            m.floor_ece,
            m.g1_verdict,
        ) {
            let (word, why) = match verdict {
                G1Verdict::Pass => ("PASS", "beats both the uncalibrated output AND the floor"),
                G1Verdict::Fail => ("FAIL", "does not beat both"),
                G1Verdict::NoClaim => (
                    "NO CLAIM",
                    "the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail",
                ),
            };
            s.push_str(&format!(
                "\n**G1 (modelless readout ECE):** raw {} · calibrated {} · conformal-naive floor {} → **{}** ({})\n",
                fmt4(raw),
                fmt4(cal),
                fmt4(floor),
                word,
                why
            ));
        }

        // typed_decisions extras.
        if suite.name == "typed_decisions" {
            if let Some(m) = suite.modelless.as_ref() {
                s.push_str(&format!(
                    "\n**typed-decisions extras (modelless):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n\n",
                    fmt_opt(m.soft_acc),
                    fmt_opt(m.brier_soft),
                    fmt_opt(m.score_mae),
                    fmt_opt(m.within_1)
                ));
                if let Some(by) = &m.by_question_type {
                    s.push_str("| model | type | n | acc | ECE(maxp) | mean conf |\n|---|---|---|---|---|---|\n");
                    for (t, h) in by {
                        s.push_str(&format!(
                            "| modelless | {t} | {} | {} | {} | {} |\n",
                            h.n,
                            fmt4(h.accuracy),
                            fmt4(h.ece),
                            fmt4(h.mean_confidence)
                        ));
                    }
                    for r in suite.laya.values() {
                        if let Some(by) = &r.by_question_type {
                            for (t, h) in by {
                                s.push_str(&format!(
                                    "| {}·{} | {t} | {} | {} | {} | {} |\n",
                                    r.lane,
                                    r.model,
                                    h.n,
                                    fmt4(h.accuracy),
                                    fmt4(h.ece),
                                    fmt4(h.mean_confidence)
                                ));
                            }
                        }
                    }
                }
            }
            for r in suite.laya.values() {
                s.push_str(&format!(
                    "**typed-decisions extras ({}·{}):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n",
                    r.lane,
                    r.model,
                    fmt_opt(r.soft_acc),
                    fmt_opt(r.brier_soft),
                    fmt_opt(r.score_mae),
                    fmt_opt(r.within_1)
                ));
            }
            if let Some(r) = &suite.clm {
                s.push_str(&format!(
                    "**typed-decisions extras ({}·{}):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n",
                    r.lane,
                    r.model,
                    fmt_opt(r.soft_acc),
                    fmt_opt(r.brier_soft),
                    fmt_opt(r.score_mae),
                    fmt_opt(r.within_1)
                ));
                if let Some(by) = &r.by_question_type {
                    for (t, h) in by {
                        s.push_str(&format!(
                            "| {}·{} | {t} | {} | {} | {} | {} |\n",
                            r.lane,
                            r.model,
                            h.n,
                            fmt4(h.accuracy),
                            fmt4(h.ece),
                            fmt4(h.mean_confidence)
                        ));
                    }
                }
            }
            if let Some(r) = &suite.gliner {
                s.push_str(&format!(
                    "**typed-decisions extras ({}·{}):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n",
                    r.lane,
                    r.model,
                    fmt_opt(r.soft_acc),
                    fmt_opt(r.brier_soft),
                    fmt_opt(r.score_mae),
                    fmt_opt(r.within_1)
                ));
                if let Some(by) = &r.by_question_type {
                    for (t, h) in by {
                        s.push_str(&format!(
                            "| {}·{} | {t} | {} | {} | {} | {} |\n",
                            r.lane,
                            r.model,
                            h.n,
                            fmt4(h.accuracy),
                            fmt4(h.ece),
                            fmt4(h.mean_confidence)
                        ));
                    }
                }
            }
            if let Some(r) = &suite.bekko {
                s.push_str(&format!(
                    "**typed-decisions extras ({}·{}):** soft_acc {} · brier_soft {} · score MAE {} · within_1 {}\n",
                    r.lane,
                    r.model,
                    fmt_opt(r.soft_acc),
                    fmt_opt(r.brier_soft),
                    fmt_opt(r.score_mae),
                    fmt_opt(r.within_1)
                ));
                if let Some(by) = &r.by_question_type {
                    for (t, h) in by {
                        s.push_str(&format!(
                            "| {}·{} | {t} | {} | {} | {} | {} |\n",
                            r.lane,
                            r.model,
                            h.n,
                            fmt4(h.accuracy),
                            fmt4(h.ece),
                            fmt4(h.mean_confidence)
                        ));
                    }
                }
            }
        }
        if matches!(suite.name.as_str(), "sst5" | "sst5_syn")
            && let Some(m) = suite.modelless.as_ref()
        {
            s.push_str(&format!(
                "\n**{} score metrics (modelless):** MAE {} · within_1 {}\n",
                suite.name,
                fmt_opt(m.score_mae),
                fmt_opt(m.within_1)
            ));
        }
        s.push('\n');
    }
    // Issue-025 landscape: PUBLISHED vendor rows, quoted as published —
    // never measured by this harness. Pin: malevrigns/agent-jev @
    // a965ca8ff06ccabc0c796dca5447b55cc2069cee (2026-09-23, Apache-2.0);
    // numbers verified against their README results table +
    // typed_decisions/agentjev_v1_report.json (/trained/*; the same file
    // carries the phase-4 pre-run baseline). Static by design — this file
    // is regenerated wholesale, so the section lives here, not in a
    // hand-edited copy.
    s.push_str(
r#"## Landscape — published specialist rows (NOT measured by this harness)

External "System One" typed-decision models on the same 400-case /
2000-question Typed Decisions official test split, quoted AS PUBLISHED
(issue 025; pin `malevrigns/agent-jev` @ `a965ca8f`, Apache-2.0). Their
protocol differs from ours — the footnotes are part of the row; no number
here is comparable without them.

| lane · model | source | acc | bool·noul / choice / score | p50 case |
|---|---|---|---|---|
| AgentJev-0.6B (598M, Qwen3-0.6B backbone) | published (their run) | **0.7925** | 88.83 / 75.33 / 75.00 | ~60–70 ms, their cuda box |
| reflex · agentjev (their service, gold-label) | **MEASURED** (bench 039, 4090) | **0.7715** | — | 88 ms (loopback HTTP) |
| Laya (published checkpoint, 421M ModernBERT) | their table — card-copied, not re-scored | 0.7700 | — | 41.53 ms (their box) |
| TypeSafe Jev 1.13.0 | their table — zero-shot generalist | 0.727 | — | — |
| reflex · laya-riir·typed | MEASURED — the typed_decisions table above | 0.7445 (baseline `aa37823`) | 78.50 / 73.33 / 72.25 | 1312 ms (m3 metal, that baseline) |
| reflex · modelless | MEASURED — the typed_decisions table above | 0.3190 (baseline `aa37823`) | 53.17 / 18.67 / 25.87 | 0.472 ms |

Footnotes: (1) their accuracy is agreement with the public TEACHER argmax;
ours is gold-label under the standard harness protocol — different
references of truth. (1b) the MEASURED agentjev row (bench 039, Issue 025
amendment 4) closes that gap for AgentJev: **0.7715 gold-label** on this
harness's split (teacher-argmax 0.7925 → gold −2.1pt) — the published
ranking SURVIVES the protocol change (+2.7pt over our measured laya-typed
0.7445; their teacher-protocol gap was +2.25). (2) their run held out 120 dev + 120 cal cases and
selected the step-600 checkpoint on dev soft-CE before opening test; ours
fits no per-benchmark head. (3) their wide-load figure (shared-prefix
298.91 ms vs unshared 609.65 ms at 66 paths / 33,547 tokens, backbone
token-ops 33,547 → 2,551 = 92.4% reduction, max prob delta 5.08e-4) is
their box and their load — not re-measured here. (4) on SHORT inputs their
own table reads Laya faster (41.53 ms vs ~60–70 ms p50/case); AgentJev's
latency win is wide candidate loads only. (5) the bool·noul column maps
their boolean primitive to our noul primitive — the closest analogue, not
a wire match; neither of their lanes carries an abstention primitive (the
wire's first-class abstention is ours alone).

"#,
    );
    // Issue-029 landscape row: fastino's fast-decisions suite — PUBLISHED
    // vendor rows, quoted as published; this harness does NOT run their
    // suite (the issue-029 non-goal: our 15 suites are the protocol).
    // Numbers verified against the fastino/GLiNER2.5-Decide model card +
    // the fastino/fast-decisions dataset card (fetched 2026-09-25,
    // Apache-2.0). The scored split is HELD OUT (their public repo ships
    // only the 100/domain dev split, with an explicit do-not-score note)
    // — so the row is necessarily quoted, never re-runnable here. Static
    // by design, the same law as the block above.
    s.push_str(
        r#"## Landscape — fast-decisions (vendor suite; NOT measured by this harness)

fastino's `fast-decisions` suite — 17 English operational-decision
domains (commerce support intent/topic, ticket routing, product feedback,
banking intent, document type, review sentiment, assistant handoff,
email triage, clinic request, travel request, news topic, paper field,
sports recap, restaurant review, benefits request, screen tags), 300
held-out test examples per domain, exact-match accuracy, the same text
and candidate labels for every model. Quoted AS PUBLISHED (issue 029).

| model | avg exact-match |
|---|---|
| GLiNER2.5-Decide (340M, DeBERTa-v3-large) | **60.2%** |
| GLiNER2.5-Decide-1B (their dataset card: "GLiNER2 XL (1B)") | 59.6% |
| JevK5 | 57.6% |
| GLiNER2.5-multi-Decide (287M) | 56.7% |
| SemIf (Qwen3.5-4B) | 56.4% |
| GLiFormer large-v1 | 49.0% |
| Laya Router | 46.6% |

Footnotes: (1) the scored split is private — their public repo carries
only the 100/domain development split with an explicit "do not report a
score computed on the files in this repo" note, so no row here can be
re-scored outside fastino. (2) their metric is per-head exact match
(single-label string equality; multi-label heads compared as sets),
averaged over the 17 domains. (3) their "Laya Router" row names no
checkpoint variant — neither the base router nor the typed specialist;
our measured laya-riir cells are the port checkpoints under OUR protocol,
so the 60.2-vs-46.6 gap is their-suite/their-checkpoint/their-protocol.
(4) our measured comparison lives in the 15-suite tables (bench 037,
`.benchmarks/037_gliner_lane_4090`): the DIRECTION is confirmed on
decision-style suites — gliner beats the laya base checkpoint 9/15
(banking77 +20.8pt, massive_intent +7.3pt, all five harness families) —
and honestly refuted on classic NLU (ag_news −24.8pt, xnli_en −38.3pt;
their card's own "not a general-purpose model" framing). The
typed_decisions headline stays laya's: the `typed` specialist 0.7445 vs
gliner 0.5280.

"#,
    );
    s
}
