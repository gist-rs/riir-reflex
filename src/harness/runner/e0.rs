//! Issue 005 (riir-instinct) E0 — the count-table EVIDENCE-DENSITY
//! measurement (T1, reflex-only), run before H2 is built.
//!
//! WHAT: per dataset suite, on the stratified selection slice (the same
//! cal-side instrument the cap/head/nb selections share — never the test
//! split, which stays reserved for T5's single read), the distribution of
//! `n` = [`NbScope::seen_count`] over the state's event stream, plus the
//! RUMOR FRACTION — states with `n < 4`, the failure class that killed
//! katgpt-rs Proposal 013's `engram_puct` at 100% (the evidence gate fired
//! on 4 of 16,930 states, so the fusion term was noise wearing a gate).
//!
//! The tables are fit on the DEPLOYED corpus — the full registry pool
//! ([`nb_doc_sets`], NB docs uncapped per label) — because the question is
//! "would the tables the shipped engine reads have evidence?", not "would
//! the selection build's tables". Scale/α/noul polarity do not move
//! seen-ness (they weight scores, not counts); the only knobs that move it
//! are the corpus set and the EVENT VIEW, so both admissible views are
//! measured (bag everywhere; pair where the suite's states carry ≥ 2
//! string fields — the same rule `build_nb_selection` uses).
//!
//! VERDICT (the issue's pre-declaration, made conservative): H2 is
//! ARMED-PENDING on a suite iff the rumor fraction ≤ 50% on ANY admissible
//! view; NOT ARMED iff every view exceeds 50%. A write-off must survive
//! both tokenizations. The view that actually binds stays nb-select's
//! train-side decision (made at T3, before any test read).
//!
//! Protocol: no gold is consumed, no test row is evaluated (prepare builds
//! case structures; E0 reads only selection-slice state strings), and the
//! measurement never arms or changes any engine posture. Report-only.

use super::*;
#[cfg(feature = "nb_scope")]
use crate::nb_scope::{NbAlpha, NbScope, NbView, view_tokens_into};

/// States per suite measured on the selection slice; `n < 4` is the rumor
/// class (Proposal 013's gate floor); > 50% rumor on a view writes H2 off
/// for that view (riir-instinct `.issues/005` §E0).
const RUMOR_N: usize = 4;
const RUMOR_CEILING: f64 = 0.5;

/// One view's evidence-density stats over the suite's selection slice.
#[derive(Debug, Clone, Serialize)]
pub struct E0ViewStats {
    /// `bag` or `pair`.
    pub view: &'static str,
    /// Selection-slice states measured.
    pub n_states: usize,
    pub min_seen: usize,
    pub p25_seen: usize,
    pub median_seen: usize,
    pub p75_seen: usize,
    pub max_seen: usize,
    pub mean_seen: f64,
    /// States with `n < 4` — the Proposal-013 rumor class.
    pub rumor_fraction: f64,
    /// States with `n == 0` (the gate could never fire).
    pub zero_fraction: f64,
    /// Mean of per-state seen/total events (the blend σ's normalizer is
    /// the total, so this is the evidence QUALITY companion).
    pub mean_seen_fraction: f64,
    /// Median total events per state (context for the σ normalizer).
    pub median_total_events: usize,
    /// Docs the tables were fit on (the deployed corpus rule).
    pub corpus_docs: usize,
    /// Distinct buckets the corpus touched at this view.
    pub observed_buckets: usize,
    /// Bucketed counts of `n`: 0, 1, 2, 3, 4-7, 8-15, 16-31, 32-63, 64+.
    pub histogram: Vec<(String, usize)>,
}

/// One suite's E0 result: per-view stats + the issue-005 pre-declaration.
#[derive(Debug, Clone, Serialize)]
pub struct E0Suite {
    pub name: String,
    pub n_states: usize,
    pub views: Vec<E0ViewStats>,
    /// The conservative pre-declaration: armed-pending iff ANY admissible
    /// view has rumor fraction ≤ [`RUMOR_CEILING`].
    pub h2_armed: bool,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct E0Meta {
    pub date_utc: String,
    pub git_sha: String,
    pub host: String,
    pub datasets_dir: String,
    /// Where the tables' counts come from.
    pub corpus_rule: &'static str,
    /// Which states are measured (and the test-split guarantee).
    pub slice_rule: &'static str,
    /// What "seen" means.
    pub seen_rule: &'static str,
    /// How the H2 pre-declaration is derived from the views.
    pub gate_rule: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct E0Output {
    pub meta: E0Meta,
    pub suites: Vec<E0Suite>,
    /// Suites that could not be measured, with the reason — loud, never a
    /// silent absence (the frontier-report law).
    pub skipped: Vec<String>,
}

/// Nearest-rank percentile of a SORTED slice: `ceil(q·n)`, 1-based. The
/// tail-support disclosure lives in `n_states` (the percentile lesson: a
/// rank without its n is an anecdote).
fn percentile(sorted: &[usize], q: f64) -> usize {
    let n = sorted.len();
    debug_assert!(n > 0);
    let idx = ((q * n as f64).ceil() as usize).clamp(1, n) - 1;
    sorted[idx]
}

/// The fixed `n` histogram buckets, in order.
const BUCKETS: [(&str, usize, usize); 9] = [
    ("0", 0, 0),
    ("1", 1, 1),
    ("2", 2, 2),
    ("3", 3, 3),
    ("4-7", 4, 7),
    ("8-15", 8, 15),
    ("16-31", 16, 31),
    ("32-63", 32, 63),
    ("64+", 64, usize::MAX),
];

fn histogram_buckets(counts: &[usize]) -> Vec<(String, usize)> {
    BUCKETS
        .iter()
        .map(|(name, lo, hi)| {
            (
                (*name).to_string(),
                counts.iter().filter(|&&c| c >= *lo && c <= *hi).count(),
            )
        })
        .collect()
}

/// The issue-005 pre-declaration over the measured views.
fn armed_pending(views: &[E0ViewStats]) -> bool {
    views.iter().any(|v| v.rumor_fraction <= RUMOR_CEILING)
}

fn view_stats(
    view: &'static str,
    seen: Vec<usize>,
    totals: &[usize],
    corpus_docs: usize,
    observed_buckets: usize,
) -> E0ViewStats {
    // Per-state pairings (seen[i] ↔ totals[i]) BEFORE any sort — the mean
    // fraction pairs each state's count with ITS OWN total. A first version
    // zipped the SORTED counts against the raw totals and read seen/total
    // > 100%, which is impossible by construction and was caught on the
    // first live run.
    let n = seen.len();
    let mean_seen_fraction = if n == 0 {
        0.0
    } else {
        seen.iter()
            .zip(totals)
            .map(|(&s, &t)| {
                debug_assert!(
                    s <= t,
                    "seen count {s} exceeds its own total {t} — pairing bug"
                );
                if t == 0 { 0.0 } else { s as f64 / t as f64 }
            })
            .sum::<f64>()
            / n as f64
    };
    let mut sorted = seen.clone();
    sorted.sort_unstable();
    let rumor = sorted.iter().filter(|&&c| c < RUMOR_N).count();
    let zero = sorted.iter().filter(|&&c| c == 0).count();
    E0ViewStats {
        view,
        n_states: n,
        min_seen: sorted.first().copied().unwrap_or(0),
        p25_seen: percentile(&sorted, 0.25),
        median_seen: percentile(&sorted, 0.5),
        p75_seen: percentile(&sorted, 0.75),
        max_seen: sorted.last().copied().unwrap_or(0),
        mean_seen: sorted.iter().sum::<usize>() as f64 / n.max(1) as f64,
        rumor_fraction: rumor as f64 / n.max(1) as f64,
        zero_fraction: zero as f64 / n.max(1) as f64,
        mean_seen_fraction,
        median_total_events: {
            let mut t = totals.to_vec();
            t.sort_unstable();
            percentile(&t, 0.5)
        },
        corpus_docs,
        observed_buckets,
        histogram: histogram_buckets(&sorted),
    }
}

/// E0 for one dataset suite: deployed-corpus tables × selection-slice
/// states, both admissible views.
fn e0_suite(spec: &SuiteSpec, dir: &Path) -> Result<E0Suite, String> {
    let prepared = prepare(spec, dir)?;
    if prepared.pool_rows.is_null() {
        return Err("no corpus-pool envelope (synthetic/code path)".to_string());
    }
    let inp = ModellessInput {
        spec,
        suite: &prepared.suite,
        train: &prepared.train,
        state_strs: &prepared.state_strs,
        cal_cases: &prepared.cal_cases,
        cal_state_strs: &prepared.cal_state_strs,
        labels: &prepared.labels,
        want_by_type: false,
        corpus_cap_per_label: spec.corpus_cap_per_label,
        head_scale: 0.0,
        head_select: false,
        nb_select: false,
        cap_source_base: "registry",
        cal_select_caps: &[],
        pool_rows: &prepared.pool_rows,
        pair_head_ab: false,
        leak_flags: None,
    };
    // The states: the stratified selection slice (the shared cal-side
    // instrument). Its content-excluded pool is NOT used — the tables fit
    // on the DEPLOYED corpus (the full pool), which is the question E0
    // answers (see the module doc).
    let SelSlice {
        cases: sel_cases,
        state_strs,
        pool: _,
    } = selection_slice(&inp, "e0")?;
    let sets = nb_doc_sets(&prepared.train, &prepared.labels, &[]);
    let corpus_docs: usize = sets.iter().map(Vec::len).sum();
    // The nb-select pair-view rule: states carrying ≥ 2 string fields.
    let pair_suite = sel_cases.first().is_some_and(|c| {
        c.state
            .as_object()
            .is_some_and(|m| m.values().filter(|v| v.is_string()).count() >= 2)
    });
    let mut views: Vec<(NbView, &'static str)> = vec![(NbView::Bag, "bag")];
    if pair_suite {
        views.push((NbView::Pair, "pair"));
    }
    // α is irrelevant to seen-ness (it weights scores, not counts);
    // ObservedLaplace is recorded as the posture the fit ran at.
    let domain_refs: Vec<&[String]> = sets.iter().map(Vec::as_slice).collect();
    let mut out = Vec::with_capacity(views.len());
    let mut toks: Vec<u32> = Vec::new();
    for (view, name) in views {
        let nb = NbScope::fit(&domain_refs, NbAlpha::ObservedLaplace, view);
        let mut seen = Vec::with_capacity(state_strs.len());
        let mut totals = Vec::with_capacity(state_strs.len());
        for s in &state_strs {
            view_tokens_into(view, s.as_bytes(), &mut toks);
            seen.push(nb.seen_count(&toks));
            totals.push(toks.len());
        }
        let stats = view_stats(name, seen, &totals, corpus_docs, nb.observed());
        eprintln!(
            "    e0: view {} → median n {} · rumor<n{} {:.1}% · seen/total {:.1}%",
            stats.view,
            stats.median_seen,
            RUMOR_N,
            stats.rumor_fraction * 100.0,
            stats.mean_seen_fraction * 100.0
        );
        out.push(stats);
    }
    let armed = armed_pending(&out);
    let verdict = if armed {
        "ARMED-PENDING".to_string()
    } else {
        format!("NOT ARMED (rumor fraction > {RUMOR_CEILING} on every admissible view)")
    };
    eprintln!("    e0: {verdict}");
    Ok(E0Suite {
        name: spec.name.to_string(),
        n_states: out.first().map_or(0, |v| v.n_states),
        views: out,
        h2_armed: armed,
        verdict,
    })
}

/// Run E0 over the requested suites (empty = every dataset suite).
/// Report-only: never touches an eval lane, never reads gold, never arms
/// the tables. Synthetic families + `code_fixtures` are skipped loud (no
/// corpus pool; the NB lane is dataset-scoped).
pub fn run_e0(opts: &RunOptions) -> Result<E0Output, String> {
    let mut suites = Vec::new();
    let mut skipped = Vec::new();
    for spec in SUITES {
        if !opts.suites.is_empty() && !opts.suites.iter().any(|s| s == spec.name) {
            continue;
        }
        if spec.synthetic.is_some() || !spec.modelless_lane {
            skipped.push(format!("{}: synthetic family / LLM-lane only — no corpus pool", spec.name));
            continue;
        }
        if spec.name == "code_fixtures" {
            skipped.push("code_fixtures: generated in-process, no corpus pool".to_string());
            continue;
        }
        match e0_suite(spec, &opts.datasets_dir) {
            Ok(s) => suites.push(s),
            Err(e) => skipped.push(format!("{}: {e}", spec.name)),
        }
    }
    if suites.is_empty() {
        return Err(format!(
            "e0: no suite measured ({} skip/failure line(s), first: {})",
            skipped.len(),
            skipped.first().map_or("none", String::as_str)
        ));
    }
    Ok(E0Output {
        meta: E0Meta {
            date_utc: iso8601_utc(),
            git_sha: git_sha().unwrap_or_else(|| "unknown".to_string()),
            host: hostname(),
            datasets_dir: opts.datasets_dir.display().to_string(),
            corpus_rule: "tables fit on the DEPLOYED corpus: the full registry pool \
                          (train minus the stratified cal front), NB docs uncapped per \
                          label (nb_doc_sets — the build_engine_with rule)",
            slice_rule: "states from the label-stratified selection slice (the shared \
                         cal-side instrument); NO gold consumed, NO test row evaluated — \
                         the test split stays reserved for T5's single read",
            seen_rule: "an event is SEEN iff its bucket was touched by any domain's \
                        training docs (the fit-time seen set); duplicates counted — the \
                        same event stream the blend σ divides by",
            gate_rule: "H2 ARMED-PENDING iff rumor fraction (n < 4) ≤ 50% on ANY \
                        admissible view; NOT ARMED iff every view exceeds it — a \
                        write-off must survive both tokenizations; the binding view \
                        stays nb-select's train-side decision",
        },
        suites,
        skipped,
    })
}

/// The markdown rendering (the record's table block).
#[must_use]
pub fn render_e0_markdown(out: &E0Output) -> String {
    let mut s = String::new();
    s.push_str("# E0 — count-table evidence density (riir-instinct Issue 005 T1)\n\n");
    s.push_str(&format!(
        "- date {} · sha {} · host {}\n- datasets {}\n",
        out.meta.date_utc, out.meta.git_sha, out.meta.host, out.meta.datasets_dir
    ));
    for (name, rule) in [
        ("corpus", out.meta.corpus_rule),
        ("slice", out.meta.slice_rule),
        ("seen", out.meta.seen_rule),
        ("gate", out.meta.gate_rule),
    ] {
        s.push_str(&format!("- {name}: {rule}\n"));
    }
    s.push_str("\n| suite | view | n | min | p25 | med | p75 | max | mean | rumor <4 | n=0 | seen/tot | med events | docs | buckets | verdict |\n");
    s.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for suite in &out.suites {
        for v in &suite.views {
            s.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {:.1} | {:.1}% | {:.1}% | {:.1}% | {} | {} | {} | {} |\n",
                suite.name,
                v.view,
                v.n_states,
                v.min_seen,
                v.p25_seen,
                v.median_seen,
                v.p75_seen,
                v.max_seen,
                v.mean_seen,
                v.rumor_fraction * 100.0,
                v.zero_fraction * 100.0,
                v.mean_seen_fraction * 100.0,
                v.median_total_events,
                v.corpus_docs,
                v.observed_buckets,
                if v.rumor_fraction <= RUMOR_CEILING { "armed-pending" } else { "not armed" },
            ));
        }
        s.push_str(&format!(
            "| **{} pre-declaration** | | | | | | | | | | | | | | | {} |\n",
            suite.name, suite.verdict
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

    #[test]
    fn nearest_rank_percentile() {
        let v: Vec<usize> = (0..10).collect();
        // n=10: p25 → ceil(2.5)=3 → idx 2; p50 → ceil(5)=5 → idx 4;
        // p75 → ceil(7.5)=8 → idx 7.
        assert_eq!(percentile(&v, 0.25), 2);
        assert_eq!(percentile(&v, 0.5), 4);
        assert_eq!(percentile(&v, 0.75), 7);
        assert_eq!(percentile(&[7], 0.5), 7, "n=1 clamps to the only value");
    }

    #[test]
    fn histogram_lands_every_bucket_edge() {
        let h = histogram_buckets(&[0, 1, 2, 3, 4, 7, 8, 15, 16, 31, 32, 63, 64, 200]);
        let get = |name: &str| h.iter().find(|(n, _)| n == name).map_or(0, |(_, c)| *c);
        assert_eq!(get("0"), 1);
        assert_eq!(get("1"), 1);
        assert_eq!(get("2"), 1);
        assert_eq!(get("3"), 1);
        assert_eq!(get("4-7"), 2);
        assert_eq!(get("8-15"), 2);
        assert_eq!(get("16-31"), 2);
        assert_eq!(get("32-63"), 2);
        assert_eq!(get("64+"), 2);
        assert_eq!(h.iter().map(|(_, c)| c).sum::<usize>(), 14);
    }

    #[test]
    fn verdict_requires_every_view_over_the_ceiling() {
        let mk = |rumor: f64| E0ViewStats {
            view: "bag",
            n_states: 100,
            min_seen: 0,
            p25_seen: 0,
            median_seen: 0,
            p75_seen: 0,
            max_seen: 0,
            mean_seen: 0.0,
            rumor_fraction: rumor,
            zero_fraction: 0.0,
            mean_seen_fraction: 0.0,
            median_total_events: 0,
            corpus_docs: 0,
            observed_buckets: 0,
            histogram: Vec::new(),
        };
        assert!(armed_pending(&[mk(0.49)]));
        assert!(armed_pending(&[mk(0.9), mk(0.3)]), "one admissible view rescues");
        assert!(!armed_pending(&[mk(0.51)]));
        assert!(!armed_pending(&[mk(0.9), mk(0.50001)]), "both over ⇒ NOT ARMED");
    }

    #[test]
    fn stats_reports_the_distribution_of_a_known_sample() {
        // counts 0..=9 → min 0, p25 2, med 4, p75 7, max 9, mean 4.5;
        // rumor (<4) = 4/10; zero = 1/10.
        let totals = vec![10usize; 10];
        let s = view_stats("bag", (0..10).collect(), &totals, 42, 7);
        assert_eq!(s.min_seen, 0);
        assert_eq!(s.p25_seen, 2);
        assert_eq!(s.median_seen, 4);
        assert_eq!(s.p75_seen, 7);
        assert_eq!(s.max_seen, 9);
        assert!((s.mean_seen - 4.5).abs() < 1e-9);
        assert!((s.rumor_fraction - 0.4).abs() < 1e-9);
        assert!((s.zero_fraction - 0.1).abs() < 1e-9);
        assert_eq!(s.corpus_docs, 42);
        assert_eq!(s.observed_buckets, 7);
    }

    #[test]
    fn seen_fraction_pairs_each_count_with_its_own_total() {
        // sorted(counts) must NOT be zipped against raw totals: with counts
        // [0, 10] and totals [10, 10] the mean fraction is (0 + 1) / 2 =
        // 0.5; pairing sorted counts [0, 10] against [10, 10] happens to
        // agree here, so use DIFFERENT totals — counts [8, 0] over totals
        // [10, 100]: correct = (0.8 + 0.0)/2 = 0.4; the sort-then-zip bug
        // pairs 0↔10 and 8↔100 → (0.0 + 0.08)/2 = 0.04.
        let s = view_stats("bag", vec![8, 0], &[10, 100], 1, 1);
        assert!((s.mean_seen_fraction - 0.4).abs() < 1e-9);
        assert!(s.mean_seen_fraction <= 1.0, "seen/total can never exceed 1");
    }
}
