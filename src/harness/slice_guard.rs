//! Slice-integrity guard (Issue 058) — the assertions over the four slices
//! every dataset-suite run composes: the test sample, the cal front, the
//! corpus pool, and (downstream of the runner) the selection slice.
//!
//! Three incidents priced this module (the owner's "we get it wrong 3rd
//! times around slices already"):
//!
//! 1. **Issue 023** — a cal-slice read on a stale/misaligned index.
//! 2. **Issue 039 T2** — the positional `cal_cap..` pool cut orphaned whole
//!    label blocks once the cal front became stratified.
//! 3. **Bench 076** — the datasets dir silently moved
//!    (`.raw/datasets_t20k` full pools → the default 4k-row pull) between
//!    the runs that selected the published postures and the run that
//!    re-published the board; every number moved with the corpus and nothing
//!    compared the pool identity ("the eval-side digest checks never caught
//!    it" — the 076 record's own words about its 4090 twin).
//!
//! What is asserted here, and how hard:
//!
//! - **Structural overlaps** (cal ∩ test, pool ∩ test, cal ∩ pool as raw
//!   dataset rows) — **hard failures**. A train mirror that carries test
//!   rows leaks; a pool that contains cal rows is the Issue-039 class.
//!   (Near-duplicates that are not verbatim stay the `slice_leak` lane's
//!   axis — exact identity is what a slice must never carry.)
//! - **Test-sample label coverage** — a hard failure when the sampling
//!   budget could have covered every engine label but did not (the
//!   Issue-039 non-representative-sample class that silently moved every
//!   number once). Disclosed-only when the budget is smaller than the label
//!   universe (coverage was never possible).
//! - **Cal-front and pool label coverage** — **disclosed facts**, not
//!   failures: a label the train pull never carried is starved data (the
//!   corpus-fallback guard, Issue 039 T3, already discloses its self-doc
//!   consequence per lane); failing the suite here would block runs on
//!   degenerate-but-real data.
//! - **Slice identity digests + counts** — recorded into results.json so a
//!   board diff can see "same pool?" mechanically. The pool digest is the
//!   RAW (uncapped) slice identity — the axis the per-lane consumed corpus
//!   digest (Issue 057) is deliberately not: at a small cap a truncated
//!   pool and a full one consume near-identical docs, which is exactly why
//!   the 076 publish sailed through.
//!
//! Pure over its inputs — no dataset dir, no engine, no feature gates. The
//! runner calls [`audit_envelopes`] at the `prepare` seam and refuses the
//! suite on any hard violation.

use serde_json::Value;
use std::collections::HashSet;

/// Per-suite minimum corpus-pool rows — the "digests tell you the pool
/// changed; a floor tells you it shrank" law (Issue 058, verdict
/// condition 1). The values sit at ~80% of the full-pull expectation (the
/// t20k re-baseline scale, `TRAIN_CAP=20000`): a 4k-row default pull —
/// the exact state that silently re-scored the board at Bench 076 —
/// REFUSES instead of publishing numbers against a starved corpus.
/// The expectation is the AUDITED quantity — the pool AFTER the cal front
/// is removed — never the raw pull: a source-limited suite must subtract
/// its cal rows first (the thai_sib200 mispin, repaired 2026-10-02 — the
/// 560 floor was 80% of a 701-row raw pull and unsatisfiable at source).
/// Suites absent from the table carry no floor (synthetic/unknown suites,
/// unit-test names). Floor violations are hard refusals like the overlaps;
/// the remedy line names the fetch command.
pub const MIN_POOL_ROWS: &[(&str, usize)] = &[
    ("ag_news", 16_000),
    ("banking77", 8_000),
    ("emotion", 12_000),
    ("massive_intent_en", 9_200),
    ("sst5", 6_800),
    ("xnli_en", 16_000),
    // Issue 080: the same TRAIN_CAP=20000 pull shape as xnli_en — pool
    // after the fixed 200-row cal front maxes at 19,800, floor = ~80%.
    ("wanli_en", 16_000),
    ("typed_decisions", 960),
    ("prompt_injections", 430),
    ("thai_wisesight", 3_200),
    // Source-measured, NOT raw-pull-derived: the suite's SOURCE yields 701
    // train rows total (re-fetch 2026-10-02, TRAIN_CAP=20000 — proven by the
    // riir-infer Plan-617 A5 board's slice refusal), so pool-after-cal maxes
    // at 501 (701 − 200 cal) and the old 560 (80% of the raw pull) was
    // unsatisfiable at any pull. 400 = 80% of 501. Both shrink classes stay
    // covered: the cal front is a fixed 200 rows, so pool = raw − 200 and a
    // raw-pull loss >14% still drops the pool below this floor. The fix is
    // decoupled from that lane's re-open (riir-infer issue 034 — the re-open
    // is a NEW pre-registered decision, never "the floor now passes").
    ("thai_sib200", 400),
];

/// The floor for one suite, if any.
#[must_use]
pub fn min_pool_rows(suite: &str) -> Option<usize> {
    MIN_POOL_ROWS
        .iter()
        .find(|(s, _)| *s == suite)
        .map(|(_, n)| *n)
}

/// One pinned known-dirty slice state (see [`SliceFacts::KNOWN_DIRTY`]).
/// Membership is the exact `(test, cal, pool)` digest triple — the same
/// digests results.json publishes, so a pin is checkable against any run
/// record without re-running anything.
#[derive(Debug, Clone)]
pub struct KnownDirtySlice {
    pub suite: &'static str,
    pub test_digest: &'static str,
    pub cal_digest: &'static str,
    pub pool_digest: &'static str,
    /// Why this dirty state is accepted, and the condition that removes
    /// the pin. Never optional: a pin without its removal condition is a
    /// loophole wearing a pin.
    pub reason: &'static str,
}

/// One suite's slice facts + audit verdict (results.json, additive;
/// `violations` empty = the hard assertions held).
#[derive(Debug, Clone, serde::Serialize)]
pub struct SliceFacts {
    /// Test-sample rows (the served eval population, raw).
    pub n_test: usize,
    /// Cal-front rows (raw).
    pub n_cal: usize,
    /// Corpus-pool rows (train minus the cal front, raw).
    pub n_pool: usize,
    /// Raw test-sample identity: fnv1a64 over the canonical row bytes,
    /// suite-scoped. Moves when the eval pull moves.
    pub test_digest: String,
    /// Raw cal-front identity (same primitive).
    pub cal_digest: String,
    /// Raw corpus-pool identity (same primitive) — the axis that catches a
    /// truncated/refetched train pull at the same cap (Bench 076's class).
    pub pool_digest: String,
    /// Engine labels absent from the test sample — non-empty is a HARD
    /// violation when the budget could have covered them (see the module
    /// doc).
    pub missing_test_labels: Vec<String>,
    /// Engine labels absent from the cal front (disclosed, not failed).
    pub missing_cal_labels: Vec<String>,
    /// Engine labels absent from the corpus pool (disclosed, not failed —
    /// the per-lane fallback guard names the self-doc consequence).
    pub missing_pool_labels: Vec<String>,
    /// Hard violations, each naming the slices and the overlapping row
    /// count. Empty = clean. The runner refuses the suite when non-empty
    /// UNLESS the exact state matches a [`SliceFacts::KNOWN_DIRTY`] pin
    /// (then `acknowledged` is true and the run proceeds, loudly).
    pub violations: Vec<String>,
    /// True when the run proceeded under a known-dirty pin (the loud
    /// acknowledgement path). Always false on clean runs — skipped in the
    /// serialization so clean records stay byte-identical to their past.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub acknowledged: bool,
}

impl SliceFacts {
    /// True when every hard assertion held (violations empty). Disclosed
    /// starvation does not dirty the verdict.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    /// The verdict's route 1 (Issue 058): a KNOWN-DIRTY slice pin — the
    /// exact `(suite, test, cal, pool)` digest triple of a frozen pool
    /// whose published records were measured on its dirty bytes. The pin
    /// is MEMBERSHIP in both directions: only that exact dirty state is
    /// accepted (same triple = same bytes, so it cannot admit a different
    /// defect), and the moment anyone repairs or re-pulls the pool the
    /// triple moves, the pin stops matching, and the refusal returns —
    /// the pin must be deleted in the same change that fixes the data.
    pub const KNOWN_DIRTY: &[KnownDirtySlice] = &[KnownDirtySlice {
        suite: "sst5",
        test_digest: "fnv1a64-67fe9666fa0c3a6f",
        cal_digest: "fnv1a64-d210e7c6707c3d40",
        pool_digest: "fnv1a64-cf933108b9825392",
        // The frozen `.raw/datasets_t20k` pull — the basis of every
        // published t20k-era row (reflex Benches 052+; the instinct
        // arena's frozen re-baseline). Aggregate blake3 over the dir is
        // pinned in .docs/02_protocols/dataset_manifest.md (ebfb0317…,
        // 8544 rows); these are the audit's own RAW slice digests of that
        // exact state. Removal condition: the t20k sst5 pool is re-pulled
        // de-duplicated under an owner re-baseline (the reflex-site
        // published numbers question rides along), never before.
        reason: "frozen t20k pool (manifest ebfb0317…, 8544 rows) carries \
                 the mirror's exact duplicates: 1 cross-split + 2 train-internal \
                 (published rows were measured on these bytes; dedupe is an \
                 owner re-baseline, instinct .issues/013 pickup)",
    }];

    /// Matches a known-dirty pin: violations present AND the exact digest
    /// triple of a pinned row (the row's `suite` must match too — the
    /// digests are suite-scoped, this is belt and braces).
    #[must_use]
    pub fn known_dirty_acknowledgement(&self, suite: &str) -> Option<&'static KnownDirtySlice> {
        if self.violations.is_empty() {
            return None;
        }
        Self::KNOWN_DIRTY.iter().find(|p| {
            p.suite == suite
                && p.test_digest == self.test_digest
                && p.cal_digest == self.cal_digest
                && p.pool_digest == self.pool_digest
        })
    }

    /// The one-line TABLES/stdout record: counts, digests, and any
    /// disclosed starvation.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let mut s = format!(
            "slice-integrity: test {} ({}) · cal {} ({}) · pool {} ({})",
            self.n_test,
            self.test_digest,
            self.n_cal,
            self.cal_digest,
            self.n_pool,
            self.pool_digest
        );
        if !self.missing_cal_labels.is_empty() {
            s.push_str(&format!(
                " · cal missing {}: {}",
                self.missing_cal_labels.len(),
                self.missing_cal_labels.join(",")
            ));
        }
        if !self.missing_pool_labels.is_empty() {
            s.push_str(&format!(
                " · pool missing {}: {}",
                self.missing_pool_labels.len(),
                self.missing_pool_labels.join(",")
            ));
        }
        if self.is_clean() {
            s.push_str(" · OK");
        } else {
            s.push_str(" · VIOLATIONS");
        }
        s
    }
}

/// fnv1a64 over a byte run — the same primitive the runner's
/// `cases_digest`/`corpus_digest` fold. Kept local on purpose: these are
/// RAW-slice digests, a different identity axis, and must not change when
/// the consumed-corpus digest's law changes.
fn fold(h: &mut u64, bytes: &[u8]) {
    for &b in bytes {
        *h ^= u64::from(b);
        *h = h.wrapping_mul(0x0010_0000_01b3);
    }
}

fn digest_prefix(h: u64) -> String {
    format!("fnv1a64-{h:016x}")
}

/// Domain-separated slice digest: seed folds the suite name and a
/// per-slice tag so identical row sets under different names/slices never
/// collide.
fn slice_digest(suite: &str, tag: u8, rows: &[&Value]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    fold(&mut h, suite.as_bytes());
    fold(&mut h, &[tag]);
    for r in rows {
        fold(&mut h, row_key(r).as_bytes());
    }
    digest_prefix(h)
}

/// Canonical bytes of one raw dataset row (compact JSON — stable per row
/// content, independent of envelope re-indexing).
fn row_key(row: &Value) -> String {
    serde_json::to_string(row).unwrap_or_else(|_| row.to_string())
}

/// Raw rows out of a split envelope (`{"rows": [{"row_idx": i, "row": …}]}`)
/// — the form `stratified_split` returns.
#[must_use]
pub fn envelope_rows(envelope: &Value) -> Vec<&Value> {
    envelope
        .get("rows")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(|w| w.get("row")).collect())
        .unwrap_or_default()
}

/// The audit over one suite's composed slices.
///
/// `labels` = the engine's label universe. `test_rows` / `cal_rows` /
/// `pool_rows` = the RAW rows behind the test sample, the cal front, and
/// the corpus pool (envelope form from `stratified_split` — all three come
/// out of one `prepare` call). `test_budget` = the sampling budget that
/// produced the test sample (0 = identity / uncapped — coverage is always
/// possible). `label_of_row` = the suite's label rule (the same
/// `train_row_label` the builders read).
///
/// Hard violations (returned in `violations`; the caller refuses the
/// suite): any raw-row overlap between the three slices, and an engine
/// label missing from the test sample while
/// `test_budget == 0 || test_budget >= labels.len()`.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn audit(
    suite: &str,
    labels: &[String],
    test_rows: &[&Value],
    cal_rows: &[&Value],
    pool_rows: &[&Value],
    test_budget: usize,
    label_of_row: &dyn Fn(&Value) -> Option<String>,
    label_coverage: bool,
) -> SliceFacts {
    let keys = |rows: &[&Value]| -> HashSet<String> { rows.iter().map(|r| row_key(r)).collect() };
    let test_keys = keys(test_rows);
    let cal_keys = keys(cal_rows);
    let pool_keys = keys(pool_rows);

    let mut violations = Vec::new();

    let cal_test = cal_keys.intersection(&test_keys).count();
    if cal_test > 0 {
        violations.push(format!(
            "{suite}: cal front ∩ test sample = {cal_test} identical row(s) — the train \
             mirror carries test rows; calibration/selection would read the answers"
        ));
    }
    let pool_test = pool_keys.intersection(&test_keys).count();
    if pool_test > 0 {
        violations.push(format!(
            "{suite}: corpus pool ∩ test sample = {pool_test} identical row(s) — the train \
             pull leaks the eval population (near-duplicates that are not verbatim are the \
             slice_leak lane's axis; EXACT identity is refused here)"
        ));
    }
    let pool_cal = pool_keys.intersection(&cal_keys).count();
    if pool_cal > 0 {
        violations.push(format!(
            "{suite}: corpus pool ∩ cal front = {pool_cal} identical row(s) — selection and \
             gate-fit would score against rows the engine also trains on"
        ));
    }

    // Pool floor (Issue 058 verdict condition 1): a pool below the suite's
    // full-pull scale is the silent-shrink class — refuse with the remedy
    // named, never score the board against a starved corpus.
    if let Some(floor) = min_pool_rows(suite)
        && pool_rows.len() < floor
    {
        violations.push(format!(
            "{suite}: corpus pool = {} row(s) < floor {floor} — the pull shrank (the \
             silent-move class); re-fetch full: TRAIN_CAP=20000 scripts/fetch_datasets.sh",
            pool_rows.len()
        ));
    }

    // Label coverage per slice. Test coverage is HARD when the budget could
    // have covered every label; cal/pool coverage is disclosed (starved
    // data is the fallback guard's disclosure, not a slice-structure fault).
    let slice_labels = |rows: &[&Value]| -> HashSet<String> {
        rows.iter().filter_map(|r| label_of_row(r)).collect()
    };
    let missing = |have: &HashSet<String>| -> Vec<String> {
        labels
            .iter()
            .filter(|l| !have.contains(*l))
            .cloned()
            .collect()
    };
    let missing_test = missing(&slice_labels(test_rows));
    // Plan 010's scoped relief: a VARIABLE-OPTION breadth suite (s1mb) arms
    // its engine over the PRESENTED option-key union, which deliberately
    // exceeds the gold space (long-tail keys are presented, rarely gold) —
    // the coverage law's premise (every engine label is realizable as a
    // gold on a full pull) does not hold there. Starved labels ride the
    // Issue-039 self-doc fallback, disclosed at build. The fields stay
    // populated (the disclosure survives); only the VIOLATION is skipped,
    // and the identity checks above are untouched.
    let coverage_possible = (test_budget == 0 || test_budget >= labels.len()) && label_coverage;
    if coverage_possible && !missing_test.is_empty() {
        violations.push(format!(
            "{suite}: test sample (budget {test_budget}) misses {} engine label(s) [{}] — \
             the round-robin sampling law cannot produce this on a pull that carries the \
             label; the eval file (or its label rule) is incomplete",
            missing_test.len(),
            missing_test.join(",")
        ));
    }
    let missing_cal = missing(&slice_labels(cal_rows));
    let missing_pool = missing(&slice_labels(pool_rows));

    SliceFacts {
        n_test: test_rows.len(),
        n_cal: cal_rows.len(),
        n_pool: pool_rows.len(),
        test_digest: slice_digest(suite, 0, test_rows),
        cal_digest: slice_digest(suite, 1, cal_rows),
        pool_digest: slice_digest(suite, 2, pool_rows),
        missing_test_labels: missing_test,
        missing_cal_labels: missing_cal,
        missing_pool_labels: missing_pool,
        violations,
        acknowledged: false,
    }
}

/// Convenience over split envelopes: extracts the raw rows and delegates to
/// [`audit`].
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn audit_envelopes(
    suite: &str,
    labels: &[String],
    test_envelope: &Value,
    cal_envelope: &Value,
    pool_envelope: &Value,
    test_budget: usize,
    label_of_row: &dyn Fn(&Value) -> Option<String>,
    label_coverage: bool,
) -> SliceFacts {
    audit(
        suite,
        labels,
        &envelope_rows(test_envelope),
        &envelope_rows(cal_envelope),
        &envelope_rows(pool_envelope),
        test_budget,
        label_of_row,
        label_coverage,
    )
}

#[cfg(test)]
mod tests {
    //! Every violation class fires; the clean path passes; the digests are
    //! deterministic and move exactly when the slice moves (the Bench-076
    //! truncated-pool class pinned as a unit test).

    use super::*;
    use serde_json::json;

    fn row(label: &str, text: &str) -> Value {
        json!({ "label": label, "text": text })
    }

    fn envelope(rows: &[Value]) -> Value {
        let wrapped: Vec<Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| json!({ "row_idx": i, "row": r }))
            .collect();
        json!({ "rows": wrapped })
    }

    fn label_of(row: &Value) -> Option<String> {
        row.get("label").and_then(Value::as_str).map(str::to_string)
    }

    fn labels() -> Vec<String> {
        vec!["a".to_string(), "b".to_string()]
    }

    #[test]
    fn clean_slices_pass_with_facts() {
        let test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(f.is_clean(), "unexpected violations: {:?}", f.violations);
        assert_eq!(f.n_test, 2);
        assert_eq!(f.n_cal, 2);
        assert_eq!(f.n_pool, 2);
        assert!(f.summary_line().contains("OK"));
    }

    #[test]
    fn cal_front_carrying_test_rows_is_refused() {
        let test = envelope(&[row("a", "shared"), row("b", "t2")]);
        let cal = envelope(&[row("a", "shared"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(f.violations.iter().any(|v| v.contains("cal front ∩ test")));
    }

    #[test]
    fn pool_containing_test_rows_is_refused() {
        let test = envelope(&[row("a", "leaked"), row("b", "t2")]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "leaked"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(f.violations.iter().any(|v| v.contains("pool ∩ test")));
    }

    #[test]
    fn pool_containing_cal_rows_is_refused() {
        let test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let cal = envelope(&[row("a", "shared"), row("b", "c2")]);
        let pool = envelope(&[row("a", "shared"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(f.violations.iter().any(|v| v.contains("pool ∩ cal")));
    }

    #[test]
    fn unrepresentative_test_sample_is_refused_when_budget_covers_labels() {
        let test = envelope(&[row("a", "t1"), row("a", "t2")]); // b never sampled
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(
            f.violations
                .iter()
                .any(|v| v.contains("misses 1 engine label"))
        );
        // The same slice at a budget too small to cover: disclosed, not failed.
        let f2 = audit_envelopes("s", &labels(), &test, &cal, &pool, 1, &label_of, true);
        assert!(f2.is_clean());
        assert_eq!(f2.missing_test_labels, vec!["b".to_string()]);
    }

    #[test]
    fn uncapped_identity_sample_requires_full_coverage() {
        let test = envelope(&[row("a", "t1")]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 0, &label_of, true);
        assert!(
            f.violations
                .iter()
                .any(|v| v.contains("misses 1 engine label"))
        );
    }

    #[test]
    fn cal_and_pool_starvation_is_disclosed_not_failed() {
        let test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let cal = envelope(&[row("a", "c1")]); // b starved in cal
        let pool = envelope(&[row("a", "p1")]); // b starved in pool
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(f.is_clean());
        assert_eq!(f.missing_cal_labels, vec!["b".to_string()]);
        assert_eq!(f.missing_pool_labels, vec!["b".to_string()]);
        assert!(f.summary_line().contains("cal missing 1: b"));
        assert!(!f.summary_line().contains("VIOLATIONS"));
    }

    #[test]
    fn pool_digest_moves_when_the_train_pull_grows() {
        // The Bench-076 class: a truncated pull and the full pull produce
        // different pools — the raw digest must see it (the consumed-corpus
        // digest at a small cap cannot).
        let test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let truncated = envelope(&[row("a", "p1"), row("b", "p2")]);
        let full = envelope(&[
            row("a", "p1"),
            row("b", "p2"),
            row("a", "p3"),
            row("b", "p4"),
        ]);
        let f1 = audit_envelopes("s", &labels(), &test, &cal, &truncated, 2, &label_of, true);
        let f2 = audit_envelopes("s", &labels(), &test, &cal, &full, 2, &label_of, true);
        assert_ne!(f1.pool_digest, f2.pool_digest);
        assert_ne!(f1.n_pool, f2.n_pool);
        assert_eq!(f1.test_digest, f2.test_digest);
        assert_eq!(f1.cal_digest, f2.cal_digest);
    }

    #[test]
    fn digests_are_deterministic_and_suite_scoped() {
        let test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let a = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        let b = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert_eq!(a.pool_digest, b.pool_digest);
        assert_eq!(a.test_digest, b.test_digest);
        assert_eq!(a.cal_digest, b.cal_digest);
        let other = audit_envelopes("other", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert_ne!(a.pool_digest, other.pool_digest);
        assert_ne!(a.test_digest, other.test_digest);
    }

    #[test]
    fn envelope_reindexing_does_not_move_the_digests() {
        // The envelope's row_idx is mechanical; the raw row content is the
        // identity. A re-indexed envelope must read identical.
        let rows = vec![row("a", "t1"), row("b", "t2")];
        let e1 = envelope(&rows);
        let shifted: Vec<Value> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| json!({ "row_idx": i + 100, "row": r }))
            .collect();
        let e2 = json!({ "rows": shifted });
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f1 = audit_envelopes("s", &labels(), &e1, &cal, &pool, 2, &label_of, true);
        let f2 = audit_envelopes("s", &labels(), &e2, &cal, &pool, 2, &label_of, true);
        assert_eq!(f1.test_digest, f2.test_digest);
        assert_eq!(f1.n_test, f2.n_test);
    }

    #[test]
    fn envelope_rows_reads_the_wrapped_form() {
        let e = envelope(&[row("a", "x")]);
        assert_eq!(envelope_rows(&e).len(), 1);
        assert_eq!(envelope_rows(&Value::Null).len(), 0);
    }

    #[test]
    fn known_dirty_pin_accepts_only_its_exact_triple() {
        // A dirty slice whose triple matches the pin → acknowledged. The
        // probe suite "s" carries no floor, so the tiny pools here trip
        // ONLY the overlap violations.
        let test = envelope(&[row("a", "shared"), row("b", "t2")]);
        let cal = envelope(&[row("a", "shared"), row("b", "c2")]);
        let pool = envelope(&[row("a", "shared"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(!f.is_clean());
        assert!(f.known_dirty_acknowledgement("s").is_none()); // not pinned
        assert!(!f.acknowledged);

        // A clean slice never consults the pin table — including for a
        // suite that HAS a pin (a repaired pool just stops matching).
        let clean_test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let clean_cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pinned_suite = SliceFacts::KNOWN_DIRTY[0].suite;
        // sst5 (the pinned suite) carries a pool floor, so the clean probe
        // there runs at floor scale: a synthetic full-scale pool is built
        // by repeating the floor-sized rows.
        let floor = min_pool_rows(pinned_suite).unwrap();
        let big_pool = envelope(
            &(0..floor)
                .map(|i| row("a", &format!("p{i}")))
                .collect::<Vec<_>>(),
        );
        let big_labels = vec!["a".to_string()]; // floor-probe pool is single-label
        let fc = audit_envelopes(
            pinned_suite,
            &big_labels,
            &clean_test,
            &clean_cal,
            &big_pool,
            2,
            &label_of,
            true,
        );
        // The pool is at floor scale and clean of overlaps; whatever label
        // starvation is disclosed, the pin itself is never consulted.
        assert!(!fc.violations.iter().any(|v| v.contains("floor")));
        assert!(fc.known_dirty_acknowledgement(pinned_suite).is_none());
    }

    #[test]
    fn pool_below_floor_is_refused_with_the_remedy() {
        // The Bench-076 class made structural: a 4k-scale pool for a
        // full-pull suite refuses, naming the fetch command.
        let test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f = audit_envelopes("sst5", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(
            f.violations
                .iter()
                .any(|v| v.contains("< floor 6800") && v.contains("TRAIN_CAP=20000"))
        );
        // At floor scale the violation disappears.
        let floor = min_pool_rows("sst5").unwrap();
        let big_pool = envelope(
            &(0..floor)
                .map(|i| row("a", &format!("p{i}")))
                .collect::<Vec<_>>(),
        );
        let one_label = vec!["a".to_string()];
        let f2 = audit_envelopes("sst5", &one_label, &test, &cal, &big_pool, 2, &label_of, true);
        assert!(!f2.violations.iter().any(|v| v.contains("floor")));
    }

    #[test]
    fn suites_without_floor_are_unfloored() {
        let test = envelope(&[row("a", "t1"), row("b", "t2")]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        assert!(f.is_clean()); // "s" is not in MIN_POOL_ROWS
        assert!(min_pool_rows("s").is_none());
    }

    #[test]
    fn every_known_dirty_pin_is_well_formed() {
        // Membership discipline: every pin names a real suite, carries
        // complete digest triples, and its reason states the removal
        // condition. A pin missing any of these is a loophole wearing a
        // pin (the reviewer's stale-acknowledgement property: a repair
        // moves the triple, the pin goes inert, and this table is where
        // the removal is performed).
        for p in SliceFacts::KNOWN_DIRTY {
            assert!(!p.suite.is_empty());
            assert!(p.test_digest.starts_with("fnv1a64-"));
            assert!(p.cal_digest.starts_with("fnv1a64-"));
            assert!(p.pool_digest.starts_with("fnv1a64-"));
            assert!(p.reason.contains("removal") || p.reason.contains("re-baseline") || p.reason.contains("owner"),
                "pin for {} must state its removal condition", p.suite);
            // Pins are unique per suite (one accepted dirty state each).
            let dups = SliceFacts::KNOWN_DIRTY.iter().filter(|q| q.suite == p.suite).count();
            assert_eq!(dups, 1, "duplicate pins for suite {}", p.suite);
        }
    }

    #[test]
    fn no_violations_when_a_row_lacks_the_label_rule() {
        // Rows the label rule cannot project (None) simply do not count
        // toward coverage — the audit must not panic or fabricate labels.
        let test = envelope(&[json!({ "text": "no label here" })]);
        let cal = envelope(&[row("a", "c1"), row("b", "c2")]);
        let pool = envelope(&[row("a", "p1"), row("b", "p2")]);
        let f = audit_envelopes("s", &labels(), &test, &cal, &pool, 2, &label_of, true);
        // Both labels missing from the test sample (nothing projected) —
        // the hard violation fires with both named, no panic.
        assert!(
            f.violations
                .iter()
                .any(|v| v.contains("misses 2 engine label(s)"))
        );
    }
}
